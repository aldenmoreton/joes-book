import { createTheme } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    primary: {
      main: '#00c853',
	  light: '#5efc82',
	  dark: '#009624'
    },
    secondary: {
      main: '#19857b',
	  light: '#e2f1f8',
	  dark: '#808e95'
    },
    error: {
		main: '#ff0000'
    },
  },
});

export default theme;