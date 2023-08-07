import { IntValue, StringValue } from "deno-environment";

export const past = new IntValue('PAST').get();
export const url = new StringValue('URL').get();
export const redisHostname = new StringValue('REDIS_HOSTNAME').get();
export const redisPort = new IntValue('REDIS_PORT').get();
