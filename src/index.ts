import express from "express";
import cors from "cors";
import ChmiApi from "./chmi-api.ts";
import { inputSchema } from "./input.ts";
import { past } from "./env.ts";
import stream from "stream";

const app = express();

app.use(cors());

app.get("/", (_req, res) => {
	res.send("Ahoj");
});

app.get("/api/get", async (_req, res) => {
	res.setHeader("Content-Type", "application/json");
	const data = await ChmiApi().getList();
	res.send(JSON.stringify(data));
});

app.get("/api/get/:year/:month/:day/:hour/:minute", async (req, res) => {
	const input = inputSchema.safeParse(req.params);
	if (!input.success) {
		res.send("Invalid address").status(403);
		return;
	}
	const requestedDate = new Date(
		input.data.year,
		input.data.month - 1,
		input.data.day,
		input.data.hour,
		input.data.minute
	);
	const requestedTime = requestedDate.getTime() / 1000 / 60;
	if (requestedTime % 10 !== 0) {
		res.send("Invalid address").status(403);
		return;
	}
	const currentDate = new Date();
	const currentTime =
		Math.round(currentDate.getTime() / 1000 / 60) +
		currentDate.getTimezoneOffset();
	if (requestedTime > currentTime || requestedTime < currentTime - (past * 10) - 10) {
	// if (requestedTime > currentTime) {
		res.send("Unsupported time").status(400);
		return;
	}
	const image = await ChmiApi().getImage(
		input.data.year,
		input.data.month,
		input.data.day,
		input.data.hour,
		input.data.minute
	);
	if (!image) {
		res.send("Failed to get image").status(500);
		return;
	}
  const readStream = new stream.PassThrough();
  readStream.end(image);

	res.setHeader("Content-Type", "image/png");
	res.setHeader(
		"Content-Disposition",
		"attachment; filename=" +
			`meteo-${input.data.year}-${input.data.month}-${input.data.day}-${input.data.hour}-${input.data.minute}.png`
	);
	res.type("png");
  readStream.pipe(res);
});

app.listen(8080, () => {
	console.log("Server is listening at http://0.0.0.0:8080");
});
