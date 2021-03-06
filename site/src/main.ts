/* eslint-disable */
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { VueCookieNext } from "vue-cookie-next";
import Notifications from "@kyvg/vue3-notification";
import { createMetaManager } from 'vue-meta'
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import "~/styles/index.scss";
import Markdown from 'vue3-markdown-it';

import VueUploadComponent from "vue-upload-component";

const app = createApp(App);
app.use(VueCookieNext);
app.use(ElementPlus);
app.use(Notifications);
app.use(router);
app.use(Markdown);

app.component("file-upload", VueUploadComponent);
app.use(createMetaManager());
app.mount("#app");

// set default config
VueCookieNext.config({ expire: "7d" });