# Purpose and scope

Accepted founding direction, 2026-11-14. Owner and acceptance authority:
Joel Overbeck. This document records the interview's product choices and the
technical decisions delegated by that interview; it does not activate implementation.

## Purpose and real consumer

Purpose class: personal utility. Give the owner a comfortable, local Ubuntu
browser application for generating and keeping images through Venice's native
image API. The immediate problems are the chat UI's blanket 1,500-character
limit, cumbersome controls and inspection, unclear model differences, and poor
access to downloadable output. The owner wants photography, graphics, anime and
other styles, with quality-oriented choices rather than a single cheapest default.

The real inputs are the owner's prompts and settings, Venice's current catalog
and API responses. The consumer is the owner composing, inspecting and reusing
images. No downstream tabletop pipeline or other repository is a dependency.
The owner identified arkham-unfolded, mudsong, netrunner-unfolded and
unfolded-tabletop as parallel work and explicitly said this project displaces none.
No claim about those projects' completion or inherited evidence is made here.

## First useful outcome

From one local launch, compose a prompt, choose an appropriate available model
with understandable guidance, set valid size and supported options, generate,
inspect the original, download it, and find the automatically saved image and
recipe after restarting. Reuse that recipe without retyping it. A refusal must
leave the prompt intact and show all useful information the provider supplied.

Accepted choices:

- Rust owns application behavior; TypeScript is a thin browser presentation layer.
- Personal Ubuntu use on loopback, with a local browser.
- Native Venice generation and a saved library first; editing/upscaling later.
- Safe mode starts off. Send that choice explicitly. No optional prompt enhancement
  or automatic rewriting, shortening, translating or expansion of the owner's text.
- Automatically preserve every returned result and its prompt/settings outside Git.
- Research and explain every currently available generation model, including
  strengths, prompt advice, declared privacy and uncensored claims, and unknowns.
- MIT license; public repository named `venice-image-studio`.

## Current boundary and uncertainty

The current authorized task is research, interview, foundations and public
publication. It includes the verbatim Desktop constitution and linked agent
instructions. It excludes app implementation, paid calls, private prompt/key
inspection and model benchmarking. Publication of these documents was explicitly
requested; it is not acceptance of unbuilt behavior.

The current bottleneck is establishing an honest provider contract and usable
model guidance. Later implementation needs a key supplied privately and bounded
real-generation verification. Unresolved claims include acceptance beyond the
generic 7,500-character schema, model-specific seed effectiveness, incomplete
refusal reasons, and provider-internal rewriting. These do not block documents;
they limit what the app can promise.

No hosted service, LAN access, accounts, direct upstream-model API routing,
autonomous prompt optimization, model leaderboard, training, cloud library,
tabletop integration or shared SDK extraction is selected. Useful refinements
must serve this same generation-to-library path; “all useful features” is not
authority for unrelated pipelines.
