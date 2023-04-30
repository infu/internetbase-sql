import { ic, Alias, Record, registerPlugin, Result, Variant, Vec, int64, nat, nat32 } from "azle";
import { execute } from "../../hello/src";

type QueryResultTypeTuple<T extends any[]> = T;
type QueryResultTypeObject<T> = T;


type SQLite = {
  query<T extends object>(sql: string, ...arg: any[]): QueryResultTypeObject<T>[];
  query_tuple<T extends any[]>(sql: string, ...arg: any[]): QueryResultTypeTuple<T>[];
  execute: (sql: string, ...arg: any[]) => nat32;
  last_id: () => nat32;
};

export const query_one = <T extends object>(sql: string, ...arg: any[]): QueryResultTypeObject<T> => {
  let r = plugin.query<T>(sql, ...arg);
  if (!r[0]) throw "Not found";
  return r[0]
}

export const me = () => ic.caller().toUint8Array();

const plugin: SQLite = registerPlugin({
  globalObjectName: "SQLite",
  rustRegisterFunctionName: "_ic_sqlite_plugin_register",
});

export const db = {
  query: plugin.query,
  query_one,
  query_tuple: plugin.query_tuple,
  execute: plugin.execute,
  last_id: plugin.last_id
}