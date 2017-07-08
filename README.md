# Plasma GIF generator

![blue waves](/gifs/U_sw-2jXBgRuEdnwfn9WFFh4V1yXIrL0GmFA1zsvI91ivTyz0Vyumqp4ikK0yytBj_jnMwXNuCCB.gif)

This is a Rust/SDL2 program for generating [plasma effect](https://en.wikipedia.org/wiki/Plasma_effect) animations.
It's a nice tool for generating animated avatars: plasmas can be saved in GIF format and are guaranteed to seamlessly loop.

Each plasma is a color-mapped interference pattern of several sinusoidal functions.
A human-guided genetic algorithm chooses the parameters for the user, which means no understanding of the math is required.
All the user has to do is tell the program what looks good and what looks bad, and the computer automatically generates new plasmas to fit.
