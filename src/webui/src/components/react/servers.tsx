import { api } from '@/api';
import { useEffect, Fragment, useState } from 'react';
import Loader from '@/components/react/loader';
import Header from '@/components/react/header';
import { version } from '../../../package.json';
import { useArray, classNames, isVersionTooFar, startDuration } from '@/helpers';

const getStatus = (remote: string, status: string): string => {
	const badge = {
		updated: 'bg-emerald-700/40 text-emerald-400',
		behind: 'bg-gray-700/40 text-gray-400',
		critical: 'bg-red-700/40 text-red-400'
	};

	if (remote == 'v0.0.0') {
		return badge['behind'];
	} else if (isVersionTooFar(version, remote.slice(1))) {
		return badge['behind'];
	} else if (remote == `v${version}`) {
		return badge['updated'];
	} else {
		return badge[status ?? 'critical'];
	}
};

const skeleton = {
	id: 'unknown',
	name: 'Loading...',
	hostname: '',
	status: 'offline',
	connection_type: 'unknown',
	connected_at: '',
	last_heartbeat: ''
};

const getServerIcon = (base: string, status: string): string => {
	return `${base}/assets/${status === 'online' ? 'ubuntu' : 'unknown'}.svg`;
};

const Index = (props: { base: string }) => {
	const items = useArray([]);

	const badge = {
		online: 'bg-emerald-400/10 text-emerald-400',
		offline: 'bg-red-500/10 text-red-500'
	};

	async function fetch() {
		items.clear();

		try {
			const agents = await api.get(props.base + '/daemon/agents/list').json();
			agents.forEach((agent: any) => {
				items.push(agent);
			});
		} catch (error) {
			console.error('Failed to fetch agents:', error);
		}
	}
	
	const removeAgent = async (id: string, name: string) => {
		if (!confirm(`Are you sure you want to remove agent "${name}"?`)) return;
		
		try {
			await api.delete(`${props.base}/daemon/agents/${id}`);
			fetch();
			alert('Agent removed successfully');
		} catch (error) {
			alert('Failed to remove agent: ' + (error as Error).message);
		}
	};

	useEffect(() => {
		fetch();
		// Refresh every 10 seconds
		const interval = setInterval(fetch, 10000);
		return () => clearInterval(interval);
	}, []);

	if (items.isEmpty()) {
		return <Loader />;
	} else {
		return (
			<Fragment>
				<Header name="Connected Agents" description="A list of all agents connected to this server.">
					<div className="flex gap-2">
						<button
							type="button"
							onClick={fetch}
							className="transition inline-flex items-center justify-center space-x-1.5 border focus:outline-none focus:ring-0 focus:ring-offset-0 focus:z-10 shrink-0 border-zinc-900 hover:border-zinc-800 bg-zinc-950 text-zinc-50 hover:bg-zinc-900 px-4 py-2 text-sm font-semibold rounded-lg">
							Refresh
						</button>
					</div>
				</Header>
				
				<table className="w-full whitespace-nowrap text-left">
					<colgroup>
						<col className="w-full sm:w-3/12" />
						<col className="lg:w-2/12" />
						<col className="lg:w-2/12" />
						<col className="lg:w-2/12" />
						<col className="lg:w-2/12" />
						<col className="lg:w-1/12" />
						<col className="lg:w-1/12" />
					</colgroup>
					<thead className="sticky top-0 z-10 bg-zinc-950 bg-opacity-75 backdrop-blur backdrop-filter border-b border-white/10 text-sm leading-6 text-white">
						<tr>
							<th scope="col" className="py-2 pl-4 pr-8 font-semibold sm:pl-6 lg:pl-8">
								Agent Name
							</th>
							<th scope="col" className="py-2 pl-0 pr-8 font-semibold table-cell">
								Hostname
							</th>
							<th scope="col" className="hidden py-2 pl-0 pr-8 font-semibold sm:table-cell">
								Agent ID
							</th>
							<th scope="col" className="hidden py-2 pl-0 pr-8 font-semibold sm:table-cell">
								Connection Type
							</th>
							<th scope="col" className="hidden py-2 pl-0 pr-8 font-semibold md:table-cell lg:pr-20">
								Last Heartbeat
							</th>
							<th scope="col" className="hidden py-2 pl-0 pr-4 font-semibold md:table-cell lg:pr-20">
								Status
							</th>
							<th scope="col" className="py-2 pl-0 pr-4 text-right font-semibold sm:pr-6 lg:pr-8">
								Actions
							</th>
						</tr>
					</thead>
					<tbody className="divide-y divide-white/5 border-b border-white/5">
						{items.value.length === 0 ? (
							<tr>
								<td colSpan={7} className="py-8 text-center text-zinc-400">
									No agents connected. Use <code className="bg-zinc-800 px-2 py-1 rounded text-sm">opm agent connect</code> to connect an agent.
								</td>
							</tr>
						) : (
							items.value.map((agent) => (
								<tr
									className="hover:bg-zinc-800/30 cursor-pointer transition"
									key={agent.id}>
									<td className="py-4 pl-4 pr-8 sm:pl-6 lg:pl-8">
										<div className="flex items-center gap-x-4">
											<img
												src={getServerIcon(props.base, agent.status)}
												className={classNames(
													agent.status === 'online' ? 'ring-emerald-400 bg-white' : 'ring-red-400 bg-red-500',
													'h-8 w-8 rounded-full ring-2'
												)}
											/>
											<div className="truncate text-sm font-medium leading-6 text-white">{agent.name}</div>
										</div>
									</td>
									<td className="py-4 pl-0 pr-4 table-cell sm:pr-8">
										<div className="text-sm leading-6 text-gray-400">{agent.hostname || 'N/A'}</div>
									</td>
									<td className="hidden py-4 pl-0 pr-4 sm:table-cell sm:pr-8">
										<div className="font-mono text-xs leading-6 text-gray-400">{agent.id.slice(0, 16)}...</div>
									</td>
									<td className="hidden py-4 pl-0 pr-4 sm:table-cell sm:pr-8">
										<div className="text-sm leading-6 text-gray-400">{agent.connection_type}</div>
									</td>
									<td className="hidden py-4 pl-0 pr-8 text-sm leading-6 text-gray-400 md:table-cell lg:pr-20">
										{agent.last_heartbeat ? new Date(agent.last_heartbeat).toLocaleString() : 'Never'}
									</td>
									<td className="py-4 pl-0 pr-4 text-sm leading-6 sm:pr-8 lg:pr-20">
										<div className="flex items-center justify-end gap-x-2 sm:justify-start">
											<div className={classNames(badge[agent.status === 'online' ? 'online' : 'offline'], 'flex-none rounded-full p-1')}>
												<div className="h-1.5 w-1.5 rounded-full bg-current" />
											</div>
											<div className="hidden text-white sm:block">{agent.status === 'online' ? 'Online' : 'Offline'}</div>
										</div>
									</td>
									<td className="py-4 pl-0 pr-4 text-right sm:pr-6 lg:pr-8" onClick={(e) => e.stopPropagation()}>
										<button
											onClick={() => removeAgent(agent.id, agent.name)}
											className="text-red-400 hover:text-red-300 text-sm font-medium transition">
											Remove
										</button>
									</td>
								</tr>
							))
						)}
					</tbody>
				</table>
			</Fragment>
		);
	}
};

export default Index;
