//@ts-nocheck

export const testQuery = query({
  args: { test: v.string() },
  handler: async (ctx, args) => {},
});

export const testQuery22 = query({
  args: { test: v.string() },
  handler: async (ctx, args) => {},
});

export const complexQuery = query({
  args: { 
    id: v.string(),
    filters: v.array(v.object({
      field: v.string(),
      value: v.any(),
      operator: v.union(v.literal("eq"), v.literal("gt"), v.literal("lt"))
    })),
    pagination: v.optional(v.object({
      limit: v.number(),
      offset: v.number()
    })),
    sort: v.union(v.literal("asc"), v.literal("desc")),
    nested: v.object({
      deep: v.object({
        deeper: v.array(v.string())
      })
    })
  },
  handler: async (ctx, args) => {},
});