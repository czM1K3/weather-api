import { opine } from "opine";
import { opineCors } from "cors";
import ChmiApi from "./chmi-api.ts";
import { inputSchema } from "./input.ts";

const app = opine();

app.use(opineCors());

app.get("/", (_req, res) => {
	res.send("Ahoj");
});

app.get("/api/data", async (_req, res) => {
	const data = await ChmiApi().getList();
	res.send(data.join("<br>"));
});

app.get("/api/get/:year/:month/:day/:hour/:minute", async (req, res) => {
	const input = inputSchema.safeParse(req.params);
	if (!input.success) {
		res.send("Invalid address").setStatus(403);
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
		res.send("Invalid address").setStatus(403);
		return;
	}
	const currentDate = new Date();
	const currentTime =
		Math.round(currentDate.getTime() / 1000 / 60) +
		currentDate.getTimezoneOffset();
	// if (requestedTime > currentTime || requestedTime < currentTime - 60) {
	if (requestedTime > currentTime) {
		res.send("Unsupported time").setStatus(400);
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
		res.send("Failed to get image").setStatus(500);
		return;
	}

	res.setHeader("Content-Type", "image/png");
	res.setHeader(
		"Content-Disposition",
		"attachment; filename=" +
			`meteo-${input.data.year}-${input.data.month}-${input.data.day}-${input.data.hour}-${input.data.minute}.png`
	);
	res.send(image);
});

app.listen(8080, () => {
	console.log("Server is listening at http://0.0.0.0:8080");
});
