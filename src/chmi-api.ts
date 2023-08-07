import { Image, decode } from "image";
import { get, save } from "./redis.ts";

const fileReg = new RegExp(
	/"(pacz2gmaps3\.z_max3d\.\d\d\d\d\d\d\d\d\.\d\d\d\d\.\d\.png)"/g
);

const f = (input: number) => {
	const str = input.toString();
	return str.length == 1 ? "0" + str : str;
};

const ChmiApi = () => {
	const getList = async () => {
		const rawData = await fetch(
			"https://www.chmi.cz/files/portal/docs/meteo/rad/inca-cz/data/czrad-z_max3d/"
		);
		const data = await rawData.text();
		const matches = data.matchAll(fileReg);
		const res: string[] = [];
		for (const match of matches) res.push(match[1]);
		return res;
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
