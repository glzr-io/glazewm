using System;
using System.Collections.Generic;
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

        /// <summary>
        /// Sends command to appropriate command handlers.
        /// </summary>
        public void Invoke<T>(T command) where T : Command
        {
            // Create a Type object representing the generic ICommandHandler type.
            var commandHandlerGeneric = typeof(ICommandHandler<>);

            // Create a Type object representing the constructed ICommandHandler generic.
            var handlerTypeToCall = commandHandlerGeneric.MakeGenericType(command.GetType());

            var handlersToCall = _registeredCommandHandlers.Where(handler => handlerTypeToCall.IsAssignableFrom(handler));

            // TODO: add centralised error handling here?
            // TODO: create CommandResponse interface with State (failure, success) and Data (any return value from command)?
            foreach (var handler in handlersToCall)
            {
                ICommandHandler<T> handlerInstance = Program.ServiceProvider.GetService(handler) as ICommandHandler<T>;
                handlerInstance.Handle(command);
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

            foreach (var handler in handlersToCall)
            {
                IEventHandler<T> handlerInstance = Program.ServiceProvider.GetService(handler) as IEventHandler<T>;
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
