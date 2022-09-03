import { InsertManyResult, MongoClient } from "mongodb"

export const addCharacters: any = async function(client: MongoClient, characters: Object[]) {
	const results = await client.db("app").collection("characters").insertMany(characters)

	return results
}

export const findCharacters: any = (client: MongoClient, query: string) => {
	client.db("app").collection("characters").find()
	return [Object()]
}