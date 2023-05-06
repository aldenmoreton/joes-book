import { MongoClient } from 'mongodb'

//TODO: Is it safe to remove this check? Github requires it to build the image
if (!process.env.MONGODB_URI) {
	throw new Error('Invalid environment variable: "MONGODB_URI"')
}

const uri = process.env.MONGODB_URI
const options = {}

let client: MongoClient
let clientPromise: Promise<MongoClient>

//TODO:
if (!process.env.MONGODB_URI) {
	throw new Error('Please add your Mongo URI to .env.local')
}

if (process.env.NODE_ENV === 'development') {
	// In development mode, use a global variable so that the value
	// is preserved across module reloads caused by HMR (Hot Module Replacement).
	if (!global._mongoClientPromise) {
		client = new MongoClient(uri, options)
		global._mongoClientPromise = client.connect()
	}
	clientPromise = global._mongoClientPromise
} else {
	// In production mode, it's best to not use a global variable.
	client = new MongoClient(uri, options)
	clientPromise = client.connect()
}

// If DB or collections in DB are not created, add them
const collectionNames = ["books", "events", "picks"]
clientPromise.then(
	(res) => {
		const db = res.db("app")
		db.listCollections({}, {nameOnly: true}).toArray().then(
			(array) => {
				let currentCollectionNames = []
				for (let i = 0; i < array.length; i++) {
					currentCollectionNames.push(array[i].name)
				}
				for (let name of collectionNames) {
					if (!currentCollectionNames.includes(name)) {
						db.createCollection(name)
					}
				}
			}
		)
	}
)

// Export a module-scoped MongoClient promise. By doing this in a
// separate module, the client can be shared across functions.
export default clientPromise
