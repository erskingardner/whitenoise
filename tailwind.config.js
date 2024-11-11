/** @type {import('tailwindcss').Config} */
export default {
    content: ["./index.html", "./src/**/*.{html,js,svelte,ts}"],
    safelist: [
        "text-2xl",
        "text-3xl",
        "text-4xl",
        "text-5xl",
        "text-6xl",
        "text-red-500",
        "text-blue-500",
        "text-green-500",
        "text-yellow-500",
        "text-gray-500",
        "text-gray-600",
        "text-gray-700",
        "text-gray-800",
    ],
    theme: {
        extend: {
            fontFamily: {
                mono: [
                    "Inconsolata",
                    "Menlo",
                    "Monaco",
                    "Ubuntu Mono",
                    "Consolas",
                    "Courier New",
                    "monospace",
                ],
            },
        },
    },
    plugins: [require("@tailwindcss/forms")],
};
