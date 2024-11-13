//@ts-nocheck

export default defineSchema({
    // test table that uses all convex data types
    // For 'any' types we will want to use recursion to generate the type
    test: defineTable({
      _id: v.id("test"),
      _null: v.null(),
      int64: v.int64(),
      float64: v.number(),
      boolean: v.boolean(),
      string: v.string(),
      bytes: v.bytes(),
      array: v.array(v.any()),
      object: v.object({
        some: v.any(),
        value: v.any(),
      }),
      record: v.record(v.any(), v.any()),
    }),
  });