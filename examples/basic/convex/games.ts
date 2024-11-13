import { query, mutation, QueryCtx } from "./_generated/server";

export const getGame = query({
    args: {},
    handler: async (ctx, args) => {
        return await getGameData(ctx)
    },
});

export const winGame = mutation({
    args: {},
    handler: async (ctx, args) => {
        let game = await getGameData(ctx)

        if (!game) {
            await ctx.db.insert("games", {
                win_count: 1,
                loss_count: 0,
            });
        } else {
            await ctx.db.patch(game._id, {
                win_count: game.win_count + 1,
            });
        }

        return game
    },
});

export const lossGame = mutation({
    args: {},
    handler: async (ctx, args) => {
        let game = await getGameData(ctx)

        if (!game) {
            await ctx.db.insert("games", {
                win_count: 0,
                loss_count: 1,
            });
        } else {
            await ctx.db.patch(game._id, {
                loss_count: game.loss_count + 1,
            });
        }

        return game
    },
});

async function getGameData(ctx: QueryCtx)
{
    return await ctx.db.query("games").first()
}
