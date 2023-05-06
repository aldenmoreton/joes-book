//TODO: Delete this and make API routes for each of these possibly

import { ObjectId } from 'mongodb'
import { MongoClient } from "mongodb"

type NewBook = {
	name: string,
	owner: ObjectId
}
export const createBook = async function(client: MongoClient, book: NewBook, currentBoooks: Array<string>) {
	const bookRes = await client.db().collection("books").insertOne(
		{
			...book,
			"CreatedDate": new Date(Date.now())
		})

	if (bookRes.acknowledged) {
		const userRes = await client.db().collection("users").updateOne({_id: new ObjectId(book.owner)}, { $push : { "books" : bookRes.insertedId} })
		return bookRes.insertedId
	}

	return null
}

export const getBookCardProps = async (client: MongoClient, bookIds: ObjectId[]) => {
	const booksPromise = client.db().collection("books").aggregate(
		[
			{$match: {_id: {$in: bookIds}}},
			{$lookup: {from: 'users', localField: 'owner', foreignField: '_id', as: 'ownerInfo'}},
			{$project: {name: 1, ownerInfo: {image: 1, name: 1}}},
			{$set: {ownerInfo: {$arrayElemAt: ["$ownerInfo", 0]}}}
		]
	)

	const books = await booksPromise.toArray()

	const cards = books.map((book) => {
		return {
			name: book.name,
			owner: book.ownerInfo.name,
			id: book._id.toString(),
			img: book.ownerInfo.image,
			notificationCount: 0
		}
	})

	return cards
}
