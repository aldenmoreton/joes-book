import type { NextApiRequest, NextApiResponse } from 'next'

export default function handler(req: NextApiRequest, res: NextApiResponse) {
	console.log('API Receiving:', req.body)
	if (req.method === 'POST') {
		res.redirect(`/books/${req.body.name}`)
	}
}