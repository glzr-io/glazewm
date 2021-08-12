using Microsoft.Extensions.DependencyInjection;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;

namespace LarsWM.Infrastructure.Bussing
{
  /// <summary>
  /// Bus facilitates communication to command and event handlers.
  /// </summary>
  public sealed class Bus : IBus
  {
    private List<Type> _registeredCommandHandlers = new List<Type>();
    private List<Type> _registeredEventHandlers = new List<Type>();
    private static readonly Object lockObj = new Object();

    /// <summary>
    /// Sends command to appropriate command handler.
    /// </summary>
    public dynamic Invoke<T>(T command) where T : Command
    {
      // Create a Type object representing the generic ICommandHandler type.
      var commandHandlerGeneric = typeof(ICommandHandler<>);

      // Create a Type object representing the constructed ICommandHandler generic.
      var handlerTypeToCall = commandHandlerGeneric.MakeGenericType(command.GetType());

      var handlers = _registeredCommandHandlers.Where(handler => handlerTypeToCall.IsAssignableFrom(handler)).ToList();

      if (handlers.Count() != 1)
      {
        throw new Exception("Only one CommandHandler can be registered to handle a Command.");
      }

      // TODO: Add centralised error handling here?
      Debug.WriteLine($"Command {command.Name} invoked.");

      ICommandHandler<T> handlerInstance = ServiceLocator.Provider.GetRequiredService(handlers[0]) as ICommandHandler<T>;
      lock (lockObj)
      {
        return handlerInstance.Handle(command);
      }
    }

    /// <summary>
    /// Sends event to appropriate event handlers.
    /// </summary>
    public void RaiseEvent<T>(T @event) where T : Event
    {
      // Create a Type object representing the generic IEventHandler type.
      var eventHandlerGeneric = typeof(IEventHandler<>);

      // Create a Type object representing the constructed IEventHandler generic.
      var handlerTypeToCall = eventHandlerGeneric.MakeGenericType(@event.GetType());

      var handlersToCall = _registeredEventHandlers.Where(handler => handlerTypeToCall.IsAssignableFrom(handler));

      Debug.WriteLine($"Event {@event.Name} emitted.");

      // TODO: Add centralised error handling here?
      foreach (var handler in handlersToCall)
      {
        IEventHandler<T> handlerInstance = ServiceLocator.Provider.GetService(handler) as IEventHandler<T>;
        handlerInstance.Handle(@event);
      }
    }

    public void RegisterCommandHandler<T>()
    {
      _registeredCommandHandlers.Add(typeof(T));
    }

    public void RegisterEventHandler<T>()
    {
      _registeredEventHandlers.Add(typeof(T));
    }
  }
}
