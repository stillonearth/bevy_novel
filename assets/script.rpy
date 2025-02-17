define character_1 = Character("Character 1", color="#000000")
define character_2 = Character("Character 2", color="#ffaabb")

label start:
    jump chapter1_1

label chapter1_1:

    scene background

    show character komarito

    komarov "I've always loved visual novels"

    komarov "Bevy seems like the perfect choice for this project"

    "I'm planning on using Rust as my programming language"

    hide character komarito

    "It's a bit intimidating, but I'm up for the challenge"

    scene inverted

    "I've already started working on some basic components"

    "But I need to make sure they're stable and bug-free first"

    "Wish you were here to help me brainstorm"

    "Thanks for listening, even if it's just a voice in my head!"

    return