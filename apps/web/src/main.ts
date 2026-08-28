import '../../../packages/design-system/tokens.css';
import './styles.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('The app mount point is missing.');
}

const app = mount(App, { target });

export default app;
