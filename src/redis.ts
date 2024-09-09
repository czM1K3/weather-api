import { createClient } from "redis";
import { redisUrl } from "./env.ts";

const redis = createClient({
  url: redisUrl,
});
redis.connect();

export const save = async (key: string, data: string) => {
 	await redis.set(key, data, {
		EX: 60 * 60,
  });
};

export const get = async (key: string) => {
  const res = await redis.get(key);
  return res ? res.toString() : null;
};
