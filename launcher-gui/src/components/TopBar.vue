<script setup>
import { ref } from 'vue'
import tauriLogo from '@/assets/tauri.ico'
import ThemeController from '@/components/ThemeController.vue'
</script>

<template>
    <div class="navbar">
        <div class="navbar-start">
            <div class="flex">
                <div class="mx-2 tooltip" data-tip="tauri">
                    <a href="https://tauri.app" target="_blank">
                        <img
                            :src="tauriLogo"
                            alt="Tauri Logo"
                            class="h-6 mr-2"
                        />
                    </a>
                </div>

                <div class="mx-2 tooltip" data-tip="vue">
                    <a href="https://vuejs.org" target="_blank">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            class="h-6 w-6"
                            viewBox="0 0 128 128"
                        >
                            <path
                                d="M0 8.934l49.854.158 14.167 24.47 14.432-24.47L128 8.935l-63.834 110.14zm126.98.637l-24.36.02-38.476 66.053L25.691 9.592.942 9.572l63.211 107.89zm-25.149-.008l-22.745.168-15.053 24.647L49.216 9.73l-22.794-.168 37.731 64.476zm-75.834-.17l23.002.009m-23.002-.01l23.002.01"
                                fill="none"
                            />
                            <path
                                d="M25.997 9.393l23.002.009L64.035 34.36 79.018 9.404 102 9.398 64.15 75.053z"
                                fill="#35495e"
                            />
                            <path
                                d="M.91 9.569l25.067-.172 38.15 65.659L101.98 9.401l25.11.026-62.966 108.06z"
                                fill="#41b883"
                            />
                        </svg>
                    </a>
                </div>
            </div>
            <div class="flex-1 flex justify-center">
                <ThemeController />
            </div>
        </div>

        <div class="navbar-center">
            <a class="text-lg font-bold"> Tauri + Vue Game Launcher</a>
        </div>

        <div class="navbar-end">
            <a
                href="https://github.com/sogladev/launcher-gui"
                target="_blank"
                class="text-md link link-primary flex items-center"
            >
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-6 w-6 mr-2"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        fill-rule="evenodd"
                        d="M12 2C6.476 2 2 6.484 2 12.018c0 4.424 2.865 8.168 6.839 9.504.5.092.682-.217.682-.484 0-.237-.008-.868-.013-1.704-2.782.605-3.369-1.342-3.369-1.342-.454-1.157-1.11-1.467-1.11-1.467-.91-.62.07-.607.07-.607 1.005.071 1.533 1.033 1.533 1.033.894 1.532 2.348 1.09 2.921.834.092-.647.35-1.09.636-1.341-2.221-.253-4.555-1.111-4.555-4.944 0-1.091.39-1.984 1.029-2.682-.103-.254-.447-1.275.098-2.656 0 0 .84-.27 2.75 1.025A9.564 9.564 0 0112 6.844a9.56 9.56 0 012.5.338c1.909-1.293 2.748-1.025 2.748-1.025.547 1.381.203 2.402.1 2.656.64.698 1.028 1.59 1.028 2.682 0 3.841-2.337 4.688-4.566 4.938.359.309.678.919.678 1.855 0 1.337-.012 2.419-.012 2.748 0 .27.18.58.688.481A10.02 10.02 0 0022 12.018C22 6.484 17.523 2 12 2z"
                        clip-rule="evenodd"
                    />
                </svg>
                Sogladev
            </a>
        </div>
    </div>
</template>

<style scoped>
/* .footer {
  position: fixed;
  left: 0;
  bottom: 0;
  width: 100%;
  text-align: center;
} */
</style>
