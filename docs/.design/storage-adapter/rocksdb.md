# Column family (CF)

We need 1 CF:
- `DB_COLUMN_FAMILY`: stores everything including records and offsets

# design choice

We only have one copy of `Record` in rocksdb but we build multiple indices on one `Record`.

# Types of KV pairs

## shard record

key: `/{namespace}/{shard}/record/{record_offset}`

value: record data

## shard offset

key: `/offset/{namespace}/{shard}`

value: **next** offset of shard `shard` under the namespace `namespace`

When a new shard is created under a namespace, a new key will be generated with value `0`.

When a shard is deleted, we will only delete the shard offset key associated with this shard. The stored record will be deleted in the background.

This key value pair will be incremented when calling `write` or `batch_write` method

## key offset

key: `/key/{namespace}/{shard}/{key}`

value: the offset of the record in shard `shard` under namespace `namespace` with `record.key = key`

Note: the key of a record should be unique under any `{namespace}/{shard}` pair

This key value pair is used for fast record retrieval in `read_by_key` method

## tag offsets

key: `/tag/{namespace}/{shard}/{tag}`

value: a list of offsets for all records in shard `shard` under namespace `namespace` whose `record.tags` contains `tag`

This key value pair is used for fast record retrieval in `read_by_tag` method

## group record offsets

key: `/group/{group}`

value: A map of (`/{namespace}/{shard}`, offset) pairs

This key value pair will be set in `commit_offset` method