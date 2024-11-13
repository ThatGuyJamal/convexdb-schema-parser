//@ts-nocheck

// https://docs.convex.dev/database/types
export default defineSchema({
    // test table that uses all convex data types and variants
    test: defineTable({
       // Basic Types
      _id: v.id("test"),
      _null: v.null(),
      _int64: v.int64(),
      _float64: v.number(),
      _boolean: v.boolean(),
      _string: v.string(),
      _bytes: v.bytes(),

      // Arrays
      _simple_array: v.array(v.string()),
      _number_array: v.array(v.number()),
      _mixed_array: v.array(v.any()),
      _nested_array: v.array(v.array(v.string())),
      _deep_nested_array: v.array(v.array(v.array(v.number()))),

      // Objects
      _simple_object: v.object({
        key: v.string(),
      }),
      _complex_object: v.object({
        string: v.string(),
        number: v.number(),
        boolean: v.boolean(),
        nullable: v.null(),
        optional: v.optional(v.string()),
      }),
      _nested_object: v.object({
        level1: v.object({
          level2: v.object({
            level3: v.string(),
          }),
        }),
      }),

      // Records
      _string_record: v.record(v.string(), v.string()),
      _number_record: v.record(v.string(), v.number()),
      _complex_record: v.record(v.string(), v.any()),

      // Unions
      _simple_union: v.union(v.string(), v.number()),
      _complex_union: v.union(
        v.string(),
        v.number(),
        v.boolean(),
        v.null(),
        v.array(v.string())
      ),
      _literal_union: v.union(
        v.literal("draft"),
        v.literal("published"),
        v.literal("archived")
      ),

      // Optionals
      _optional_string: v.optional(v.string()),
      _optional_object: v.optional(v.object({
        key: v.string(),
      })),
      _optional_union: v.optional(v.union(v.string(), v.number())),
      _optional_array: v.optional(v.array(v.string())),
      
      // Complex objects
      _complex_nested: v.object({
        metadata: v.object({
          tags: v.array(v.string()),
          counts: v.record(v.string(), v.number()),
          status: v.union(v.literal("active"), v.literal("inactive")),
        }),
        settings: v.optional(v.object({
          preferences: v.record(v.string(), v.any()),
          limits: v.array(v.number()),
        })),
        nested_arrays: v.array(v.array(v.union(v.string(), v.number()))),
      }),
    }),
  });