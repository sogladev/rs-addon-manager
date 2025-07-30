<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
const CAROUSEL_INTERVAL_IN_MILLISECONDS = 15000

const images = [
    {
        src: new URL(`/src/assets/images/image-1280x720-0.jpg`, import.meta.url)
            .href,
        title: 'The Pantheon',
        caption:
            'A group of titans who shaped and ordered worlds, creating the titan-forged and empowering the Dragon Aspects to protect Azeroth.',
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-1.jpg`, import.meta.url)
            .href,
        title: "Opening of the Gates of Ahn'Qiraj",
        caption:
            "The grand event where both Alliance and Horde forces collaborated to gather resources for the war effort, culminating in the ringing of the Scarab Gong to unseal the ancient gates of Ahn'Qiraj, leading to massive battles against the Qiraji forces.",
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-2.jpg`, import.meta.url)
            .href,
        title: 'Tyrande Whisperwind',
        caption:
            "The revered High Priestess of Elune and leader of the Night Elves, known for her unwavering dedication to her people and her pivotal role in Azeroth's history.",
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-3.jpg`, import.meta.url)
            .href,
        title: 'Arthas Menethil',
        caption:
            "Once the noble Crown Prince of Lordaeron and a valiant paladin, Arthas's tragic quest to save his people led him to embrace darkness, ultimately becoming the feared Lich King.",
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-4.jpg`, import.meta.url)
            .href,
        title: 'Ossirian the Unscarred',
        caption:
            "The formidable final boss of the Ruins of Ahn'Qiraj, a Horusath known for his near-invulnerability, challenging adventurers to strategic combat within the ancient temple.",
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-5.jpg`, import.meta.url)
            .href,
        title: 'Opening of the Dark Portal',
        caption:
            'The monumental event marking the beginning of the Burning Crusade expansion, where the Dark Portal reopened, allowing heroes to traverse into the shattered realm of Outland.',
    },
    {
        src: new URL(`/src/assets/images/image-1280x720-6.jpg`, import.meta.url)
            .href,
        title: 'Illidan Stormrage',
        caption:
            'The enigmatic Night Elf sorcerer turned Demon Hunter, known as "The Betrayer," who seized control of the Black Temple in Outland, waging a personal war against the Burning Legion.',
    },
]

const currentSlideIndex = ref(0)
let autoplayTimer: number | undefined

function nextSlide() {
    currentSlideIndex.value = (currentSlideIndex.value + 1) % images.length
}

function previousSlide() {
    const i = currentSlideIndex.value
    currentSlideIndex.value = i === 0 ? images.length - 1 : i - 1
}

function start() {
    if (autoplayTimer === undefined) {
        autoplayTimer = window.setInterval(
            nextSlide,
            CAROUSEL_INTERVAL_IN_MILLISECONDS
        )
    }
}

function pause() {
    if (autoplayTimer) {
        clearInterval(autoplayTimer)
        autoplayTimer = undefined
    }
}

onMounted(() => {
    start()
})

onUnmounted(() => {
    pause()
})

watch(currentSlideIndex, (newIndex) => {
    // Update the slide ID based on the current slide index
    const slideId = `slide${newIndex + 1}`
    document.getElementById(slideId)?.scrollIntoView({ behavior: 'smooth' })
})
</script>

<template>
    <div class="carousel w-full" @mouseover="pause" @mouseleave="start">
        <!-- Dynamic slides with title and caption -->
        <div
            v-for="(image, i) in images"
            :key="i"
            :id="`slide${i + 1}`"
            class="carousel-item relative w-full flex"
        >
            <img :src="image.src" class="w-2/3" />
            <div class="w-1/3 p-4">
                <h2 class="text-xl font-bold">{{ image.title }}</h2>
                <p class="mt-2">{{ image.caption }}</p>
            </div>

            <div
                class="carousel-control absolute left-5 right-5 top-1/2 flex -translate-y-1/2 transform justify-between"
            >
                <a @click="previousSlide" class="btn btn-circle">❮</a>
                <a @click="nextSlide" class="btn btn-circle">❯</a>
            </div>
        </div>
    </div>
</template>

<style lang="css" scoped>
.carousel .carousel-control {
    visibility: hidden;
}

.carousel:hover .carousel-control {
    visibility: visible;
}
</style>
