defmodule ImprintorTest do
  use ExUnit.Case
  doctest Imprintor

  test "greets the world" do
    assert Imprintor.hello() == :world
  end
end
