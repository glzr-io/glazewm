using Microsoft.Extensions.DependencyInjection;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Reactive.Subjects;

namespace GlazeWM.Infrastructure.Bussing
{
  /// <summary>
  /// Bus facilitates communication to command and event handlers.
  /// </summary>
  public sealed class Bus
  {
    public readonly Subject<Event> Events = new Subject<Event>();
    private static readonly Object _lockObj = new Object();

    /// <summary>
    /// Sends command to appropriate command handler.
    /// </summary>
    public CommandResponse Invoke<T>(T command) where T : Command
    {
      Debug.WriteLine($"Command {command.Name} invoked.");

      // Create a `Type` object representing the constructed `ICommandHandler` generic.
      var handlerType = typeof(ICommandHandler<>).MakeGenericType(command.GetType());

      var handlerInstance = ServiceLocator.Provider.GetRequiredService(handlerType)
        as ICommandHandler<T>;

      lock (_lockObj)
      {
        return handlerInstance.Handle(command);
      }
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void RaiseEvent<T>(T @event) where T : Event
    {
      Debug.WriteLine($"Event {@event.Name} emitted.");

      // Create a `Type` object representing the constructed `IEventHandler` generic.
      var handlerType = typeof(IEventHandler<>).MakeGenericType(@event.GetType());

      var handlerInstances = ServiceLocator.Provider.GetServices(handlerType)
        as IEnumerable<IEventHandler<T>>;

      foreach (var handler in handlerInstances)
      {
        lock (_lockObj)
        {
          handler.Handle(@event);
        }
      }

      // Emit event through subject.
      Events.OnNext(@event);
    }
  }
}
