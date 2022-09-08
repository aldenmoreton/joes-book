import { useScrollTrigger } from "@mui/material"
import { MongoClient, ObjectId } from "mongodb"
import { useRowSelect } from "react-table"

export const addUser = async function(client: MongoClient, characters: Object[]) {
	const results = await client.db("app").collection("characters").insertMany(characters)

	return results
}

export const findUser = (client: MongoClient, query: string) => {
	client.db("app").collection("characters").find()
	return [Object()]
}

export const getUserById = async (client: MongoClient, userIds: ObjectId[]) => {
	const users = await client.db("app").collection("books")
	.find({_id: {$in: userIds}})

	return await users.toArray()
}
