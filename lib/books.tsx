import { ObjectId } from 'mongodb'
import { MongoClient } from "mongodb"

export const createBook = async function(client: MongoClient, book: Object[]) {
	let results = await client.db().collection("books").insertOne(book)

	return results
}

export const getBooksById = async (client: MongoClient, bookIds: ObjectId[]) => {
	const books = await client.db().collection("books")
	.find({_id: {$in: bookIds}})

	return await books.toArray()
}

export const getBookCardProps = async (client: MongoClient, bookIds: ObjectId[]) => {
	const books = await client.db().collection("books").aggregate(
		[
			{$match: {_id: {$in: bookIds}}},
			{$lookup: {from: 'users', localField: 'owner.userId', foreignField: '_id', as: 'ownerInfo'}},
			{$project: {name: 1, owner: {name: 1}, ownerInfo: {image: 1}}}
		]
	).toArray()

	const cards = books.map((book) => {
		return {name: book.name, owner: book.owner.name, id: book._id.toString(), img: book.ownerInfo[0].image}
	})

	return cards
}
