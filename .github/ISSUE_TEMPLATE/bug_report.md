name: I think I found a bug
description: Create a bug-report issue
labels: [bug]
body:

- type: textarea
  id: description
  validations:
  required: true
  attributes:
  label: Description
  description: Clear and concise description of what the bug is
- type: textarea
  id: steps-to-reproduce
  validations:
  required: true
  attributes:
  label: Reproduction steps
  description: instructions on reproducing the bug (such as a screenshot)
- type: input
  id: version
  attributes:
  label: Browser / device
  description: where did you run into this issue?
  validations:
  required: true
