import { computed } from 'vue'
import { storeToRefs } from 'pinia'

import { useWalletStore } from '@/stores/wallet'
import { useNotificationStore } from '@/stores/notifications'

export function useWallet() {
  const walletStore = useWalletStore()
  const notificationStore = useNotificationStore()

  const {
    isConnecting,
    isConnected,
    address,
    chainId,
    balance,
    walletType,
    formattedAddress,
    networkName,
    isMainnet
  } = storeToRefs(walletStore)

  // Computed properties for UI
  const connectionStatus = computed(() => {
    if (isConnecting.value) return 'connecting'
    if (isConnected.value) return 'connected'
    return 'disconnected'
  })

  const canTrade = computed(() => {
    return isConnected.value && isMainnet.value
  })

  const needsNetworkSwitch = computed(() => {
    return isConnected.value && !isMainnet.value
  })

  // Wallet actions
  async function connectWallet(providerType?: string) {
    try {
      await walletStore.connect(providerType)

      notificationStore.notifySystem(
        'Wallet Connected',
        `Successfully connected ${walletType.value} on ${networkName.value}`,
        'success'
      )
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error occurred'

      notificationStore.notifySystem(
        'Connection Failed',
        `Failed to connect wallet: ${message}`,
        'error'
      )

      throw error
    }
  }

  async function disconnectWallet() {
    try {
      const prevWalletType = walletType.value
      await walletStore.disconnect()

      notificationStore.notifySystem(
        'Wallet Disconnected',
        `${prevWalletType} has been disconnected`,
        'info'
      )
    } catch (error) {
      notificationStore.notifySystem(
        'Disconnect Failed',
        'Failed to disconnect wallet properly',
        'error'
      )
    }
  }

  async function switchToMainnet() {
    if (!isConnected.value) return

    try {
      // Default to Ethereum mainnet
      await walletStore.switchChain(1)

      notificationStore.notifySystem(
        'Network Switched',
        'Successfully switched to Ethereum mainnet',
        'success'
      )
    } catch (error) {
      notificationStore.notifySystem(
        'Network Switch Failed',
        'Failed to switch to mainnet. Please switch manually.',
        'error'
      )
    }
  }

  async function switchToArbitrum() {
    if (!isConnected.value) return

    try {
      await walletStore.switchChain(42161)

      notificationStore.notifySystem(
        'Network Switched',
        'Successfully switched to Arbitrum',
        'success'
      )
    } catch (error) {
      notificationStore.notifySystem(
        'Network Switch Failed',
        'Failed to switch to Arbitrum. Please switch manually.',
        'error'
      )
    }
  }

  async function addTokenToWallet(tokenAddress: string, symbol: string, decimals = 18, image?: string) {
    if (!isConnected.value || !window.ethereum) {
      throw new Error('Wallet not connected')
    }

    try {
      await window.ethereum.request({
        method: 'wallet_watchAsset',
        params: {
          type: 'ERC20',
          options: {
            address: tokenAddress,
            symbol,
            decimals,
            image
          }
        }
      })

      notificationStore.notifySystem(
        'Token Added',
        `${symbol} has been added to your wallet`,
        'success'
      )
    } catch (error) {
      notificationStore.notifySystem(
        'Failed to Add Token',
        `Could not add ${symbol} to wallet`,
        'error'
      )
      throw error
    }
  }

  async function signMessage(message: string): Promise<string> {
    if (!isConnected.value) {
      throw new Error('Wallet not connected')
    }

    try {
      const signature = await walletStore.signMessage(message)

      notificationStore.notifySystem(
        'Message Signed',
        'Successfully signed message',
        'success'
      )

      return signature
    } catch (error) {
      notificationStore.notifySystem(
        'Signing Failed',
        'Failed to sign message',
        'error'
      )
      throw error
    }
  }

  // Utility functions
  function getExplorerUrl(txHash?: string, address?: string): string {
    const explorerUrls: Record<number, string> = {
      1: 'https://etherscan.io',
      137: 'https://polygonscan.com',
      42161: 'https://arbiscan.io',
      10: 'https://optimistic.etherscan.io',
      8453: 'https://basescan.org'
    }

    const baseUrl = explorerUrls[chainId.value] || explorerUrls[1]

    if (txHash) return `${baseUrl}/tx/${txHash}`
    if (address) return `${baseUrl}/address/${address}`
    return baseUrl
  }

  function formatBalance(balance: string, decimals = 4): string {
    const num = parseFloat(balance)
    if (num === 0) return '0'
    if (num < 0.001) return '< 0.001'
    return num.toFixed(decimals)
  }

  function isValidAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address)
  }

  // Auto-initialization
  async function initialize() {
    try {
      await walletStore.autoConnect()
    } catch (error) {
      console.warn('Auto-connect failed:', error)
    }
  }

  return {
    // State
    isConnecting,
    isConnected,
    address,
    chainId,
    balance,
    walletType,
    formattedAddress,
    networkName,
    isMainnet,

    // Computed
    connectionStatus,
    canTrade,
    needsNetworkSwitch,

    // Actions
    connectWallet,
    disconnectWallet,
    switchToMainnet,
    switchToArbitrum,
    addTokenToWallet,
    signMessage,

    // Utilities
    getExplorerUrl,
    formatBalance,
    isValidAddress,
    initialize
  }
}