import { IntValue, StringValue } from "deno-environment";

export const past = new IntValue('PAST').get();
export const url = new StringValue('URL').get();
