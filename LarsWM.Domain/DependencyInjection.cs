using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.CommandHandlers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.Commands;
using LarsWM.Domain.Monitors.EventHandler;
using LarsWM.Domain.Monitors.Events;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.UserConfigs.CommandHandlers;
using LarsWM.Domain.UserConfigs.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Windows.CommandHandlers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Windows.EventHandlers;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.CommandHandlers;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using LarsWM.Infrastructure.WindowsApi.Events;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain
{
  public static class DependencyInjection
  {
    public static IServiceCollection AddDomainServices(this IServiceCollection services)
    {
      services.AddSingleton<ContainerService>();
      services.AddSingleton<MonitorService>();
      services.AddSingleton<UserConfigService>();
      services.AddSingleton<WindowService>();
      services.AddSingleton<WorkspaceService>();

      services.AddTransient<ICommandHandler<AttachContainerCommand>, AttachContainerHandler>();
      services.AddTransient<ICommandHandler<ChangeContainerLayoutCommand>, ChangeContainerLayoutHandler>();
      services.AddTransient<ICommandHandler<ChangeFocusedContainerLayoutCommand>, ChangeFocusedContainerLayoutHandler>();
      services.AddTransient<ICommandHandler<DetachContainerCommand>, DetachContainerHandler>();
      services.AddTransient<ICommandHandler<RedrawContainersCommand>, RedrawContainersHandler>();
      services.AddTransient<ICommandHandler<ReplaceContainerCommand>, ReplaceContainerHandler>();
      services.AddTransient<ICommandHandler<SetFocusedDescendantCommand>, SetFocusedDescendantHandler>();
      services.AddTransient<ICommandHandler<SwapContainersCommand>, SwapContainersHandler>();
      services.AddTransient<ICommandHandler<AddMonitorCommand>, AddMonitorHandler>();
      services.AddTransient<ICommandHandler<EvaluateUserConfigCommand>, EvaluateUserConfigHandler>();
      services.AddTransient<ICommandHandler<RegisterKeybindingsCommand>, RegisterKeybindingsHandler>();
      services.AddTransient<ICommandHandler<AddInitialWindowsCommand>, AddInitialWindowsHandler>();
      services.AddTransient<ICommandHandler<AddWindowCommand>, AddWindowHandler>();
      services.AddTransient<ICommandHandler<CloseFocusedWindowCommand>, CloseFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<FocusInDirectionCommand>, FocusInDirectionHandler>();
      services.AddTransient<ICommandHandler<FocusWindowCommand>, FocusWindowHandler>();
      services.AddTransient<ICommandHandler<MoveFocusedWindowCommand>, MoveFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<RemoveWindowCommand>, RemoveWindowHandler>();
      services.AddTransient<ICommandHandler<ResizeFocusedWindowCommand>, ResizeFocusedWindowHandler>();
      services.AddTransient<ICommandHandler<AttachWorkspaceToMonitorCommand>, AttachWorkspaceToMonitorHandler>();
      services.AddTransient<ICommandHandler<CreateWorkspaceCommand>, CreateWorkspaceHandler>();
      services.AddTransient<ICommandHandler<DetachWorkspaceFromMonitorCommand>, DetachWorkspaceFromMonitorHandler>();
      services.AddTransient<ICommandHandler<DisplayWorkspaceCommand>, DisplayWorkspaceHandler>();
      services.AddTransient<ICommandHandler<FocusWorkspaceCommand>, FocusWorkspaceHandler>();
      services.AddTransient<ICommandHandler<MoveFocusedWindowToWorkspaceCommand>, MoveFocusedWindowToWorkspaceHandler>();

      services.AddTransient<IEventHandler<MonitorAddedEvent>, MonitorAddedHandler>();
      services.AddTransient<IEventHandler<WindowDestroyedEvent>, WindowDestroyedHandler>();
      services.AddTransient<IEventHandler<WindowFocusedEvent>, WindowFocusedHandler>();
      services.AddTransient<IEventHandler<WindowShownEvent>, WindowShownHandler>();

      return services;
    }
  }
}
