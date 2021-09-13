using System;

public class FatalUserException : Exception
{
  public FatalUserException(string message) : base(message)
  {
  }
}
