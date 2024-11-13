import { v } from "convex/values";
import { query, mutation, QueryCtx } from "./_generated/server";

export const getGame = query({
    args: {},
    handler: async (ctx, args) => {
        return await getGameData(ctx)
    },
});

export const saveGame = mutation({
    args: {
        win_count: v.optional(v.number()),
        loss_count: v.optional(v.number()),
    },
    handler: async (ctx, args) => {
        let game = await getGameData(ctx)

        if (!game) {
            await ctx.db.insert("games", {
                win_count: args.win_count ?? 0,
                loss_count: args.loss_count ?? 0,
            });
        } else {
            const updateData: any = {};
            if (args.win_count !== undefined) updateData.win_count = args.win_count + (game.win_count ?? 0);
            if (args.loss_count !== undefined) updateData.loss_count = args.loss_count + (game.loss_count ?? 0);
            await ctx.db.patch(game._id, updateData);
        }
    },
});

async function getGameData(ctx: QueryCtx)
{
    return await ctx.db.query("games").first()
}
