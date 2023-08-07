import { z } from "zod";

export const inputSchema = z.object({
	year: z.coerce.number().min(2000).max(9999),
	month: z.coerce.number().min(1).max(12),
	day: z.coerce.number().min(1).max(31),
	hour: z.coerce.number().min(0).max(23),
	minute: z.coerce.number().min(0).max(50).multipleOf(10),
});
