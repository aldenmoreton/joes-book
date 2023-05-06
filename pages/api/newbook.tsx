import type { NextApiRequest, NextApiResponse } from 'next'

import { authOptions } from "./auth/[...nextauth]"
import { unstable_getServerSession } from "next-auth/next"

import clientPromise from "../../lib/mongodb"
import { createBook } from '../../lib/books'
import { ObjectId } from 'mongodb'

//TODO: Type
//TODO: Security checks
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
	if (req.method === 'POST') {
		const session: any = await unstable_getServerSession(req, res, authOptions)

		const client = await clientPromise

		const newBook = {
			name: req.body.name,
			owner: new ObjectId(session.user.id)
		}

		const insertID = await createBook(client, newBook, session.user.books)
		if (insertID) {
			res.redirect(`/books/${insertID}`)
		}

		res.status(500).json('Failed to create book')
	}
}