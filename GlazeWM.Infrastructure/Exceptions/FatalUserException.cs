using System;

namespace GlazeWM.Infrastructure.Exceptions
{
  public class FatalUserException : Exception
  {
    public FatalUserException()
    {
    }

    public FatalUserException(string message) : base(message)
    {
    }

    public FatalUserException(
      string message,
      Exception innerException) : base(message, innerException)
    {
    }
  }
}
