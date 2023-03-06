Feature: PubNub publish feature

  Background:
    Given the demo keyset

  @contract=publishWithSpaceIdAndMessageType
  Scenario: Simple publish
    When I publish a message
    Then I get a timetoken 1234567890
