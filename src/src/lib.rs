use rusqlite::{ToSql, Statement};
use rusqlite::types::{Value, Type};
use boa_engine::JsValue;
use boa_engine::JsString;
use boa_engine::JsResult;
use boa_engine::JsError;
use boa_engine::JsObject;
use boa_engine::object::builtins::JsArray;
use boa_engine::object::builtins::JsUint8Array;
use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::object::builtins::JsTypedArray;
use rusqlite::params;

fn _ic_sqlite_plugin_register(boa_context: &mut boa_engine::Context) {
    let SQLite = boa_engine::object::ObjectInitializer::new(boa_context)
        .function(
            boa_engine::NativeFunction::from_fn_ptr(_ic_sqlite_plugin_execute),
            "execute",
            0,
        )
        .function(
            boa_engine::NativeFunction::from_fn_ptr(_ic_sqlite_plugin_query_tuple),
            "query_tuple",
            0,
        )
        .function(
            boa_engine::NativeFunction::from_fn_ptr(_ic_sqlite_plugin_query),
            "query",
            0,
        )
        .function(
            boa_engine::NativeFunction::from_fn_ptr(_ic_sqlite_plugin_last_insert_id),
            "last_id",
            0,
        )
        .build();

    boa_context.register_global_property("SQLite", SQLite, boa_engine::property::Attribute::all());
}


fn parse_parameters(aargs: &[boa_engine::JsValue], context: &mut boa_engine::Context) -> Vec<Option<Box<dyn ToSql>>> {
    aargs
        .iter()
        .skip(1)
        .map(|value| {
            if value.is_null_or_undefined() {
                None
            } else {
                let param_value = jsvalue_to_sqlite_value(value, context);
                Some(Box::new(param_value) as Box<dyn ToSql>)
            }
        })
        .collect()
}

fn _ic_sqlite_plugin_last_insert_id(
    _this: &boa_engine::JsValue,
    aargs: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    let conn = ic_sqlite::CONN.lock().unwrap();
    let last_inserted_id = conn.last_insert_rowid();
    Ok(last_inserted_id.into())
}

fn _ic_sqlite_plugin_execute(
    _this: &boa_engine::JsValue,
    aargs: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> boa_engine::JsResult<boa_engine::JsValue> {
    let sql: String = aargs[0].clone().try_from_vm_value(&mut *context).unwrap();

    let conn = ic_sqlite::CONN.lock().unwrap();
    let mut stmt = conn.prepare(&sql).map_err(|e| {
        JsError::from_opaque(format!("Failed to prepare SQL statement: {:?}", e).into())
    })?;

    let params = parse_parameters(aargs, context);
    let transformed_params = transform_params(&params);

    match stmt.execute(&*transformed_params) {
        Ok(size) => {
            let last_inserted_id = conn.last_insert_rowid();
            return Ok(last_inserted_id.into());

        }
        Err(err) => {
            return Err(JsError::from_opaque(format!("Error executing SQL query: {:?}", err).into()));
        },
    };
}

fn _ic_sqlite_plugin_query(
    _this: &boa_engine::JsValue,
    aargs: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> JsResult<JsValue> {
    let sql: String = aargs[0].clone().try_from_vm_value(&mut *context).unwrap();

    let conn = ic_sqlite::CONN.lock().unwrap();
    let mut stmt = conn.prepare(&sql).map_err(|e| {
        JsError::from_opaque(format!("Failed to prepare SQL statement: {:?}", e).into())
    })?;
    let cnt = stmt.column_count();

    // Store column names in a Vec<String> before creating `rows`
    let column_names: Vec<String> = (0..cnt)
        .map(|idx| stmt.column_name(idx).unwrap().to_string())
        .collect();

    let params = parse_parameters(aargs, context);

    let transformed_params = transform_params(&params);

    let mut rows = stmt.query(&*transformed_params).map_err(|e| {
        JsError::from_opaque(format!("Failed to execute SQL query: {:?}", e).into())
    })?;
    
    let mut res = JsArray::new(context);

    loop {
        match rows.next() {
            Ok(row) => match row {
                Some(row) => {
                    let mut js_row = JsObject::default();
                    for idx in 0..cnt {
                        let column_name = &column_names[idx];
                        let v = row.get_ref_unwrap(idx);
                        js_row.set(column_name.as_str(), sqlite_value_to_jsvalue(v, context), false, context).unwrap();
                    }
                    res.push(JsValue::from(js_row), context)?;
                }
                None => break,
            },
            Err(err) => {
                return Err(JsError::from_opaque(format!("Error executing SQL query: {:?}", err).into()));
            }
        }
    }

    Ok(res.into())
}

fn _ic_sqlite_plugin_query_tuple(
    _this: &boa_engine::JsValue,
    aargs: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> JsResult<JsValue> {
    let sql: String = aargs[0].clone().try_from_vm_value(&mut *context).unwrap();

    let conn = ic_sqlite::CONN.lock().unwrap();
    let mut stmt = conn.prepare(&sql).map_err(|e| {
        JsError::from_opaque(format!("Failed to prepare SQL statement: {:?}", e).into())
    })?;
    let cnt = stmt.column_count();
        
    let params = parse_parameters(aargs, context);

    let transformed_params = transform_params(&params);

    let mut rows = stmt.query(&*transformed_params).map_err(|e| {
        JsError::from_opaque(format!("Failed to execute SQL query: {:?}", e).into())
    })?;
    
    let mut res = JsArray::new(context);

    loop {
        match rows.next() {
            Ok(row) => match row {
                Some(row) => {
                    let mut js_row = JsArray::new(context);
                    for idx in 0..cnt {
                        let v = row.get_ref_unwrap(idx);
                        js_row.push(sqlite_value_to_jsvalue(v, context), context)?;

                    }
                    res.push(js_row, context)?;
                }
                None => break,
            },
            Err(err) => {
                return Err(JsError::from_opaque(format!("Error executing SQL query: {:?}", err).into()));
            }
        }
    }

    Ok(res.into())
}

macro_rules! params_to_slice {
    ($params:expr, $($idx:expr),+) => {
        [
            $(
                $params.get($idx).unwrap().as_ref().unwrap().as_ref(),
            )+
        ]
    };
}

// execute needs parameters from different types to be passed as tuple
fn transform_params(params: &[Option<Box<dyn ToSql>>]) -> Box<[&dyn ToSql]> {
    match params.len() {
        0 => Box::new([]),
        1 => Box::new(params_to_slice!(params, 0)),
        2 => Box::new(params_to_slice!(params, 0, 1)),
        3 => Box::new(params_to_slice!(params, 0, 1, 2)),
        4 => Box::new(params_to_slice!(params, 0, 1, 2,3)),
        5 => Box::new(params_to_slice!(params, 0, 1, 2,3,4)),
        6 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5)),
        7 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6)),
        8 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7)),
        9 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,8)),
        10 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9)),
        11 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10)),
        12 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11)),
        13 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12)),
        14 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13)),
        15 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14)),
        16 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14,15)),
        17 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14,15,16)),
        18 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14,15,16,17)),
        19 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14,15,16,17,18)),
        20 => Box::new(params_to_slice!(params, 0, 1, 2,3,4,5,6,7,9,10,11,12,13,14,15,16,17,18,19)),

        // Add more cases up to 30
        _ => panic!("Too many parameters"),
    }
}

fn sqlite_value_to_jsvalue(value: rusqlite::types::ValueRef, context: &mut boa_engine::Context) -> boa_engine::JsValue {
    use boa_engine::JsValue;
    use rusqlite::types::Type;

    match value.data_type() {
        Type::Null => JsValue::Null,
        Type::Integer => JsValue::Integer(value.as_i64().unwrap() as i32),
        Type::Real => JsValue::Rational(value.as_f64().unwrap()),
        Type::Text => {
            let text = value.as_str().unwrap().to_owned();
            JsValue::String(boa_engine::JsString::from(text))
        }
        Type::Blob => {
            let blob_data = value.as_blob().unwrap();
            let array_buffer = JsArrayBuffer::from_byte_block(blob_data.to_vec(), context).unwrap();
            JsUint8Array::from_array_buffer(array_buffer, context).unwrap().into()
        }
    }
}


fn jsvalue_to_sqlite_value(value: &JsValue, context: &mut boa_engine::Context) -> Value {

    match value {
        JsValue::Null => Value::Null,
        JsValue::Boolean(boolean) => Value::Integer(if *boolean { 1 } else { 0 }),
        JsValue::Integer(int) => Value::Integer(*int as i64),
        JsValue::Rational(num) => Value::Real(*num),
        JsValue::String(string_value) => {
             let string_ref = string_value.to_std_string().unwrap_or("".to_string());
            Value::Text(string_ref)
        }
        JsValue::Object(v) => {
            if v.is_typed_uint8_array() {
                let ju = JsUint8Array::from_object(v.clone()).unwrap();
                let length = ju.length(context).unwrap();
                let mut bytes = Vec::with_capacity(length);

                for i in 0..length {
                    let jsval = ju.at(i as i64, context).unwrap();
                    let uint8 = jsval.to_uint8(context).unwrap();
                    bytes.push(uint8);
                }

                Value::Blob(bytes)
            } else {
                panic!("Object not supported in query arguments. Only Uint8Array")
            }
        }
   
        _ => Value::Null,
    }
}
