import { useState, useCallback } from 'react';
import type { Toast, ToastType } from './toast';

const generateToastId = () => `toast-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

export const useToast = () => {
	const [toasts, setToasts] = useState<Toast[]>([]);

	const showToast = useCallback((message: string, type: ToastType = 'info', duration?: number) => {
		const id = generateToastId();
		const newToast: Toast = { id, message, type, duration };
		
		setToasts((prev) => [...prev, newToast]);
	}, []);

	const closeToast = useCallback((id: string) => {
		setToasts((prev) => prev.filter((toast) => toast.id !== id));
	}, []);

	return {
		toasts,
		showToast,
		closeToast,
		success: (message: string, duration?: number) => showToast(message, 'success', duration),
		error: (message: string, duration?: number) => showToast(message, 'error', duration),
		info: (message: string, duration?: number) => showToast(message, 'info', duration)
	};
};
