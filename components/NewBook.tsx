import FormControl from '@mui/material/FormControl';
import { Button, InputLabel } from '@mui/material';
import Input from '@mui/material/Input';
import FormHelperText from '@mui/material/FormHelperText';

export default function NewBook({ }) {
	return (
		<form action='/api/newbook' method='post'>
			<FormControl>
				<InputLabel htmlFor="name">Book Name</InputLabel>
				<Input id="name" aria-describedby="my-helper-text" type='text' name='name'/>
				<Button type='submit'>Submit</Button>
			</FormControl>
		</form>
	)
}