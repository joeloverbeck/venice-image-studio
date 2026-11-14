# Venice Image Studio

The local workflow for composing image requests, inspecting their results and
keeping the information needed to revisit them.

## Language

**Draft**:
The editable prompt and generation choices being prepared for a future attempt.
Reusing a recipe creates a draft without changing the earlier recipe.

**Generation attempt**:
One explicitly requested submission to Venice, including its outcome even when
it returns no image or its remote outcome remains unknown.
_Avoid_: Job, when that implies a recoverable remote job.

**Result**:
An image returned by a generation attempt, together with its known status.
An attempt may return multiple results or none.

**Recipe**:
The preserved account of an attempt's original prompt, submitted choices,
relevant model information and known outcome, used to understand or reuse it.
It does not promise that repeating the choices produces an identical image.

**Library**:
The owner's local collection of results and attempt records, including recipes
and failed or incomplete attempts.

**Original**:
The image exactly as returned by Venice, distinct from a thumbnail or other
derived preview.
