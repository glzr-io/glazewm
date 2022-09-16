using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using System;
using System.Collections.Generic;
using System.Reactive.Subjects;

namespace GlazeWM.Infrastructure.Bussing
{
  /// <summary>
  /// Bus facilitates communication to command and event handlers.
  /// </summary>
  public sealed class Bus
  {
    public readonly Subject<Event> Events = new();
    public readonly object LockObj = new();
    private readonly ILogger<Bus> _logger;

    public Bus(ILogger<Bus> logger)
    {
      _logger = logger;
    }

    /// <summary>
    /// Sends command to appropriate command handler.
    /// </summary>
    public CommandResponse Invoke<T>(T command) where T : Command
    {
      _logger.LogDebug("Command {commandName} invoked.", command.Name);

      // Create a `Type` object representing the constructed `ICommandHandler` generic.
      var handlerType = typeof(ICommandHandler<>).MakeGenericType(command.GetType());

      var handlerInstance = ServiceLocator.Provider.GetRequiredService(handlerType)
        as ICommandHandler<T>;

      lock (LockObj)
      {
        return handlerInstance.Handle(command);
      }
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void RaiseEvent<T>(T @event) where T : Event
    {
      // Create a `Type` object representing the constructed `IEventHandler` generic.
      var handlerType = typeof(IEventHandler<>).MakeGenericType(@event.GetType());

      var handlerInstances = ServiceLocator.Provider.GetServices(handlerType)
        as IEnumerable<IEventHandler<T>>;

      foreach (var handler in handlerInstances)
      {
        lock (LockObj)
        {
          handler.Handle(@event);
        }
      }

      // Emit event through subject.
      Events.OnNext(@event);
    }
  }
}
