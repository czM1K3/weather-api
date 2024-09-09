const pastRaw = process.env['PAST'];
export const past = parseInt(pastRaw ?? "10");
export const url = process.env['URL'];
export const redisUrl = process.env['REDIS_URL'];
