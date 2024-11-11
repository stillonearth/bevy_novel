define character_1 = Character("Character 1", color="#000000")
define character_2 = Character("Character 2", color="#ffaabb")

label start:
    jump chapter1_1

label chapter1_1:

    scene background

    "I've always loved visual novels"

    play music "Truth.mp3"

    show character komarito

    character_1 "Bevy seems like the perfect choice for this project"

    stop music

    character_1 "I'm planning on using Rust as my programming language"

    play music "Calamity.wav"

    character_1 "It's a bit intimidating, but I'm up for the challenge"

    scene city road anime

    play sound "applause.wav"

    "I've already started working on some basic components"

    show character igor

    character_2 "But I need to make sure they're stable and bug-free first"

    character_2 "Wish you were here to help me brainstorm"

    stop music fadeout 5.9

    character_2 "Thanks for listening, even if it's just a voice in my head!"

    return