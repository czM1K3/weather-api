import { Image, decode } from "imagescript";
import { get, save } from "./redis.ts";
import { url, past } from "./env.ts"

const f = (input: number) => {
	const str = input.toString();
	return str.length == 1 ? "0" + str : str;
};

const ChmiApi = () => {
	const getList = async () => {
		const currentDate = new Date();
		const lastPossibleTime = Math.floor(currentDate.getTime() / 1000 / 60 / 10 ) * 10 * 60 * 1000;
		let lastPossibleDate = new Date(lastPossibleTime);

		const possibleImage = await getImage(
			lastPossibleDate.getUTCFullYear(),
			lastPossibleDate.getUTCMonth() + 1,
			lastPossibleDate.getUTCDate(),
			lastPossibleDate.getUTCHours(),
			lastPossibleDate.getUTCMinutes()
		);
		if (!possibleImage)
			lastPossibleDate = new Date(lastPossibleDate.getTime() - 10 * 60 * 1000);

		const arr: {url: string, label: string}[] = [];
		for (let i = 0; i < past; i++) {
			const date = new Date(lastPossibleDate.getTime() - i * 10 * 60 * 1000);
			arr.push({
				label: date.toLocaleTimeString("cs-CZ", {
					timeZone: "Europe/Prague",
					timeStyle: "short"
				}),
				url: `${url}/api/get/${date.getUTCFullYear()}/${f(
					date.getUTCMonth()+1
				)}/${f(date.getUTCDate())}/${f(date.getUTCHours())}/${f(date.getUTCMinutes())}`,
			});
		}

		return arr;
	};

	const getImage = async (
		year: number,
		month: number,
		day: number,
		hour: number,
		minute: number
	) => {
		try {
			const key = `${year}-${month}-${day}-${hour}-${minute}`;
			const cached = await get(key);
			if (cached) {
				const parsed = Uint8Array.from(JSON.parse(cached));
				return parsed;
			}

			const rawData = await fetch(
				`https://www.chmi.cz/files/portal/docs/meteo/rad/inca-cz/data/czrad-z_max3d/pacz2gmaps3.z_max3d.${year}${f(
					month
				)}${f(day)}.${f(hour)}${f(minute)}.0.png`
			);
			const data = new Uint8Array(await rawData.arrayBuffer());
			const image = await decode(data) as Image;
			const finalImage = await image.crop(1, 95, 597, 320).encode();
			const stringified = JSON.stringify(Array.from(finalImage));
			await save(key, stringified);
			return finalImage;
		} catch {
			return null;
		}
	};

	return {
		getList,
		getImage,
	};
};

export default ChmiApi;
