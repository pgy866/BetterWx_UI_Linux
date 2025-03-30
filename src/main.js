import { createApp } from "vue";
import App from './App.vue'
import PrimeVue from "primevue/config";
import ToastService from 'primevue/toastservice';
import themeConfig from './assets/primevue-config.js'
import './assets/styles.css';
const app = createApp(App);
app.use(PrimeVue, themeConfig);
app.use(ToastService);
app.mount("#app");
 