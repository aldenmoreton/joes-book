import FormControl from '@mui/material/FormControl';
import { Button, InputLabel } from '@mui/material';
import Input from '@mui/material/Input';

//TODO: Add interface for hadleSubmit function
const handleSubmit = async (event: any) => {
	event.preventDefault()
	console.log('Event', event)
	const data = {
		name: event.target.name.value,
		time: '2014-01-22T14:56:59.301'
	}
	// Send the data to the server in JSON format.
	const JSONdata = JSON.stringify(data)

	// API endpoint where we send form data.
	const endpoint = '/api/newbook'

	// Form the request for sending data to the server.
	const options = {
	  // The method is POST because we are sending data.
	  method: 'POST',
	  // Tell the server we're sending JSON.
	  headers: {
		'Content-Type': 'application/json',
	  },
	  // Body of the request is the JSON data we created above.
	  body: JSONdata,
	}

	// Send the form data to our forms API on Vercel and get a response.
	const response = await fetch(endpoint, options)

	// Get the response data from server as JSON.
	// If server returns the name submitted, that means the form works.
	console.log('Response:', response)
	console.log('Redirected:', response.redirected)
	// alert(`Is this your full name: ${result.data}`)
}

export default function NewBook({ }) {
	return (
		// <form onSubmit={handleSubmit}>
		<form action="/api/newbook" method="post">
			<FormControl>
				<InputLabel htmlFor="name">Book Name</InputLabel>
				<Input id="name" aria-describedby="my-helper-text" type='text' name='name' required/>
				<Button type='submit'>Submit</Button>
			</FormControl>
		</form>
	)
}