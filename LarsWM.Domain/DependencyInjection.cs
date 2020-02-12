using LarsWM.Domain.Common.Services;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Monitors.CommandHandlers;
using LarsWM.Domain.Monitors.EventHandler;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.UserConfigs.CommandHandlers;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.CommandHandlers;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain
{
    public static class DependencyInjection
    {
        public static IServiceCollection AddDomainServices(this IServiceCollection services)
        {
            services.AddSingleton<KeybindingService>();
            services.AddSingleton<MonitorService>();
            services.AddSingleton<UserConfigService>();
            services.AddSingleton<WindowService>();
            services.AddSingleton<WorkspaceService>();
            services.AddSingleton<AddMonitorHandler>();
            services.AddSingleton<MonitorAddedHandler>();
            services.AddSingleton<ReadUserConfigHandler>();
            services.AddSingleton<CreateWorkspaceHandler>();

            return services;
        }
    }
}
