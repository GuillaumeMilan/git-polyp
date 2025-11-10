# Configure ExUnit
ExUnit.start(
  #exclude: [:slow, :integration],
  timeout: 60_000,
  max_cases: 4
)

# Load test support modules
Code.require_file("support/git_test_helper.ex", __DIR__)
Code.require_file("support/assertions.ex", __DIR__)
