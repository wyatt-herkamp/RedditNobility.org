<template>
  <metainfo>
  </metainfo>
  <Navbar :user="userStore.state.user" />

  <router-view :key="$route.fullPath" />
  <notifications position="bottom right" />
</template>
<script lang="ts">
import userStore from "@/store/user";
import Navbar from "@/components/Navbar.vue";
import router from "@/router";
import { defineComponent, onBeforeMount, onMounted } from "vue";
import { useMeta } from "vue-meta";
export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    useMeta({
      title: "Reddit Nobility",
      htmlAttrs: { lang: "en", amp: true, charset: "UTF-8" },
      meta: [
        {
          property: "og:image",
          content: "https://redditnobility.org/TuxCrown.png",
        },
        {
          property: "og:title",
          content: "Reddit Nobility",
        },
        { property: "og:type", content: "website" },
      ],
    });
    onBeforeMount(userStore.getUser);
    return { userStore };
  },
});
</script>
<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

#nav {
  padding: 30px;
}

#nav a {
  font-weight: bold;
  color: #2c3e50;
}

#nav a.router-link-exact-active {
  color: #42b983;
}
</style>
