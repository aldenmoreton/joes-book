import type { NextApiRequest, NextApiResponse } from 'next'

export default function handler(req: NextApiRequest, res: NextApiResponse) {
	console.log('Body', req.body)
	if (req.method === 'POST') {
		res.redirect(`/books/${req.body.name}`)
	}
}