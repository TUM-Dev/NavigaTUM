name: I have a feature request
description: Create a feature-request issue
labels: [enhancement]
body:

- type: textarea
  id: feature-description
  validations:
  required: true
  attributes:
  label: Description
  description: A clear and concise description of what the problem is
  placeholder: You should add ...
- type: textarea
  id: solution
  validations:
  required: true
  attributes:
  label: Prefered solution
  description: A clear and concise description of what you want to happen.
  placeholder: In my use, ...
