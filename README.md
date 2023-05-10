# InternetBase CaelumCore Sql Azle Plugin

## Usage

```js
type Category = Record<{
  id: nat32;
  name: text;
  parent_id: Opt<nat32>;
  image_url: Opt<text>;
}>;

$query;
export function get_categories(): Vec<Category> {
  return db.query<Category>(`SELECT id, name, parent_id, image_url FROM categories`);
}

```

Warning: Make sure the column names and count in your SQL query matches the record fields, or you will get obscure errors.

For example, if your query asks for `name_another` in `SELECT id, name_another, parent_id, image_url` and your record expects `name`, it will throw error.

```js

db.query_one<Order>(`SELECT service_id, prompt, buyer_id, seller_id, status, chat, rated, created_at, updated_at, price, completed_at FROM orders WHERE id = ?1 and score = ?2 and rating = ?3 LIMIT 1`, id, score, rating);

```

Warning: When you are passing parameters, you need to make sure the number of requested parameters with ?1 or ?2 or ?3 etc.. matches the count and type of passed parameters, or you will get obscure errors.

```js

let [one, two, three] = db.query_tuple<[nat32, nat64, float64]>("SELECT id, score, rating FROM user WHERE principal = ?", me());

```

db.query_tuple will return array/(ts tuple) instead of object with field names.