//@ts-nocheck

export default defineSchema({
    messages: defineTable({
      body: v.string(),
      user: v.id("users"),
    }),
    users: defineTable({
      name: v.string(),
      tokenIdentifier: v.string(),
    }).index("by_token", ["tokenIdentifier"]),
  });