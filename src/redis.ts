import { connect } from "redis";

const redis = await connect({
	hostname: "localhost",
	port: 6379,
});

export const save = async (key: string, data: string) => {
	await redis.set(key, data, {
		ex: 60 * 60,
	});
};

export const get = async (key: string) => {
	const res = await redis.get(key);
	return res ? res.toString() : null;
};
