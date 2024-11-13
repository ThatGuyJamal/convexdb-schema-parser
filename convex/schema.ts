//@ts-nocheck

// https://docs.convex.dev/database/types
export default defineSchema({
    // test table that uses all convex data types
    // For 'any' types we will want to use recursion to generate the type
    test: defineTable({
      _id: v.id("test"),
      _null: v.null(),
      _int64: v.int64(),
      _float64: v.number(),
      _boolean: v.boolean(),
      _string: v.string(),
      _bytes: v.bytes(),
      _array: v.array(v.any()),
      _nested_array: v.array(v.array(v.any())),
      _object: v.object({
        some: v.string(),
        value: v.any(),
      }),
      _nested_object: v.object({
        deep: v.object({
          deeper: v.array(v.string())
        })
      }),
      _record: v.record(v.string(), v.string()),
      _union: v.union(v.string(), v.number(), v.boolean()),
      _literal: v.union(v.literal("asc"), v.literal("desc")),
    }),
  });