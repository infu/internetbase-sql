# InternetBase CaelumCore Sql Azle Plugin

## Install

Add this package in your existing Azle 0.16.0+ project. 
(Earlier version don't support plugins)
If you don't have one -> https://github.com/demergent-labs/azle

```bash
npm i internetbase-sql
```

```js
import { db, me } from "internetbase-sql";
```

Note: You may no be able to compile your Azle+SQL project on Mac.
If you have problems, this Discord channel may help you out
https://discord.com/channels/748416164832608337/956466775380336680


## Usage

Blast where you can check if SQL will get things done for you: https://jglts-daaaa-aaaai-qnpma-cai.raw.ic0.app/99.b73c50bc49754964b43f3fcd547c025e31426d65b51d12ffebf080ae

You may want to open up two canister functions and use Blast to create and populate your db like this:
```js
let iclocal = icblast({local:true}); 
// if dfx replica is on a different ip, add local_host: "http://192.168.0.179:8000"
// if CORS won't let you connect, install CORS unblock Chrome extension
// and use the .raw Blast url https://jglts-daaaa-aaaai-qnpma-cai.raw.ic0.app/

let can = await icblast("rrkah-fqaaa-aaaaa-aaaaq-cai");
await can.execute("CREATE TABLE .....").then(log)
```



```
$update;
export function execute(q: string): nat32 {
  return db.execute(q);
}

$query;
export function query(q: string): string {
  let x;
  try {
    x = db.query(q);
  } catch (e) {
    return e as string
  }
  return JSON.stringify(x);
}

```

```js
type Category = Record<{
  id: nat32;
  name: text;
  parent_id: Opt<nat32>;
  image_url: Opt<text>;
}>;

$query;
export function get_categories(): Vec<Category> {
  return db.query<Category>(`SELECT id, name, parent_id,
   image_url FROM categories`);
}

```

Warning: Make sure the column names and count in your SQL query matches the record fields, or you will get obscure errors.

For example, if your query asks for `name_another` in `SELECT id, name_another, parent_id, image_url` and your record expects `name`, it will throw error.

```js

db.query_one<Order>(`SELECT service_id, prompt, buyer_id, seller_id,
 status, chat, rated, created_at, updated_at, price, completed_at
  FROM orders WHERE id = ?1 and score = ?2 and
   rating = ?3 LIMIT 1`, id, score, rating);

```

Warning: When you are passing parameters, you need to make sure the number of requested parameters with ?1 or ?2 or ?3 etc.. matches the count and type of passed parameters, or you will get obscure errors.

```js

let [one, two, three] = db.query_tuple<[nat32, nat64, float64]>("SELECT id,
 score, rating FROM user WHERE principal = ?", me());

```

db.query_tuple will return array/(ts tuple) instead of object with field names.