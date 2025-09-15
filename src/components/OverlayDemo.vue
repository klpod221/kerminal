<template>
  <div class="p-6 bg-[#1a1a1a] min-h-screen">
    <div class="max-w-4xl mx-auto">
      <h1 class="text-2xl font-bold text-white mb-6">Overlay System Demo</h1>

      <!-- Debug Info -->
      <div class="mb-6 p-4 bg-gray-800 rounded-lg">
        <h3 class="text-lg font-semibold text-white mb-2">Debug Info</h3>
        <div class="text-sm text-gray-300 space-y-1">
          <div>Active Overlay: <span class="text-blue-400">{{ activeOverlay?.config.id || 'None' }}</span></div>
          <div>Has Active: <span class="text-blue-400">{{ hasActiveOverlay }}</span></div>
          <div>History: <span class="text-blue-400">{{ overlayStore.history.join(' → ') || 'Empty' }}</span></div>
          <div>Total Overlays: <span class="text-blue-400">{{ overlayStore.overlays.size }}</span></div>
        </div>
      </div>

      <!-- Control Buttons -->
      <div class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <Button
            @click="openOverlay('drawer-users')"
            class="h-12"
          >
            Open Users Drawer
          </Button>

          <Button
            @click="openOverlay('modal-settings')"
            class="h-12"
            variant="secondary"
          >
            Open Settings Modal
          </Button>

          <Button
            @click="closeAllOverlays"
            class="h-12"
            variant="danger"
          >
            Close All
          </Button>
        </div>

        <!-- Hierarchy Demo -->
        <div class="bg-gray-800 p-4 rounded-lg">
          <h3 class="text-lg font-semibold text-white mb-3">Hierarchy Demo</h3>
          <div class="space-y-2 text-sm text-gray-300">
            <div>1. Click "Open Users Drawer" → Opens main drawer</div>
            <div>2. Inside drawer, click "Add User" → Opens modal (child of drawer)</div>
            <div>3. Inside modal, click "View Profile" → Opens another modal (child of add modal)</div>
            <div>4. Close any level → automatically opens parent</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Users Drawer -->
    <Drawer
      id="drawer-users"
      title="Users Management"
      :icon="Users"
      icon-background="bg-blue-500/20"
      icon-color="text-blue-400"
      width="lg"
    >
      <div class="p-4 space-y-4">
        <div class="text-white">
          <h3 class="text-lg font-semibold mb-2">User List</h3>
          <div class="space-y-2">
            <div v-for="user in demoUsers" :key="user.id"
                 class="p-3 bg-gray-800 rounded-lg flex justify-between items-center">
              <div>
                <div class="font-medium text-white">{{ user.name }}</div>
                <div class="text-sm text-gray-400">{{ user.email }}</div>
              </div>
              <div class="space-x-2">
                <Button
                  size="sm"
                  @click="openOverlay('modal-user-info', { user })"
                >
                  View Info
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  @click="openOverlay('modal-edit-user', { user })"
                >
                  Edit
                </Button>
              </div>
            </div>
          </div>
        </div>

        <div class="pt-4 border-t border-gray-700">
          <Button
            @click="openOverlay('modal-add-user')"
            class="w-full"
          >
            <Plus class="w-4 h-4 mr-2" />
            Add New User
          </Button>
        </div>
      </div>
    </Drawer>

    <!-- Add User Modal -->
    <Modal
      id="modal-add-user"
      parent-id="drawer-users"
      title="Add New User"
      :icon="UserPlus"
      icon-background="bg-green-500/20"
      icon-color="text-green-400"
      size="md"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Name</label>
          <input
            v-model="newUser.name"
            type="text"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
            placeholder="Enter user name"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Email</label>
          <input
            v-model="newUser.email"
            type="email"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
            placeholder="Enter email address"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Role</label>
          <select
            v-model="newUser.role"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
          >
            <option value="user">User</option>
            <option value="admin">Admin</option>
            <option value="moderator">Moderator</option>
          </select>
        </div>

        <div class="pt-4 border-t border-gray-700">
          <Button
            @click="openOverlay('modal-user-profile')"
            variant="secondary"
            class="mb-3 w-full"
          >
            <Eye class="w-4 h-4 mr-2" />
            Preview Profile
          </Button>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-add-user')">
          Cancel
        </Button>
        <Button @click="saveUser">
          <Save class="w-4 h-4 mr-2" />
          Save User
        </Button>
      </template>
    </Modal>

    <!-- Edit User Modal -->
    <Modal
      id="modal-edit-user"
      parent-id="drawer-users"
      title="Edit User"
      :icon="Edit"
      icon-background="bg-yellow-500/20"
      icon-color="text-yellow-400"
      size="md"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Name</label>
          <input
            v-model="editUser.name"
            type="text"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Email</label>
          <input
            v-model="editUser.email"
            type="email"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-2">Role</label>
          <select
            v-model="editUser.role"
            class="w-full px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-white focus:ring-2 focus:ring-blue-500"
          >
            <option value="user">User</option>
            <option value="admin">Admin</option>
            <option value="moderator">Moderator</option>
          </select>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-edit-user')">
          Cancel
        </Button>
        <Button @click="updateUser">
          <Save class="w-4 h-4 mr-2" />
          Update User
        </Button>
      </template>
    </Modal>

    <!-- User Info Modal -->
    <Modal
      id="modal-user-info"
      parent-id="drawer-users"
      title="User Information"
      :icon="Info"
      icon-background="bg-purple-500/20"
      icon-color="text-purple-400"
      size="lg"
    >
      <div class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Name</label>
            <div class="text-white">{{ selectedUser?.name || 'N/A' }}</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Email</label>
            <div class="text-white">{{ selectedUser?.email || 'N/A' }}</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Role</label>
            <div class="text-white capitalize">{{ selectedUser?.role || 'N/A' }}</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Status</label>
            <div class="text-green-400">Active</div>
          </div>
        </div>

        <div class="pt-4 border-t border-gray-700">
          <Button
            @click="openOverlay('modal-location-detail')"
            variant="secondary"
            class="w-full"
          >
            <MapPin class="w-4 h-4 mr-2" />
            View Location Details
          </Button>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-user-info')">
          Close
        </Button>
        <Button @click="openOverlay('modal-edit-user', { user: selectedUser })">
          <Edit class="w-4 h-4 mr-2" />
          Edit User
        </Button>
      </template>
    </Modal>

    <!-- User Profile Modal (Child of Add User) -->
    <Modal
      id="modal-user-profile"
      parent-id="modal-add-user"
      title="Profile Preview"
      :icon="User"
      icon-background="bg-indigo-500/20"
      icon-color="text-indigo-400"
      size="md"
    >
      <div class="space-y-4">
        <div class="text-center">
          <div class="w-20 h-20 bg-gray-700 rounded-full mx-auto mb-4 flex items-center justify-center">
            <User class="w-10 h-10 text-gray-400" />
          </div>
          <h3 class="text-xl font-semibold text-white">{{ newUser.name || 'New User' }}</h3>
          <p class="text-gray-400">{{ newUser.email || 'No email set' }}</p>
          <p class="text-sm text-gray-500 capitalize">{{ newUser.role || 'user' }}</p>
        </div>

        <div class="bg-gray-800 p-4 rounded-lg">
          <h4 class="font-medium text-white mb-2">Profile Summary</h4>
          <p class="text-gray-300 text-sm">
            This is how the user profile will appear after creation.
            You can make changes before saving.
          </p>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-user-profile')">
          Back to Form
        </Button>
      </template>
    </Modal>

    <!-- Location Detail Modal (Child of User Info) -->
    <Modal
      id="modal-location-detail"
      parent-id="modal-user-info"
      title="Location Details"
      :icon="MapPin"
      icon-background="bg-red-500/20"
      icon-color="text-red-400"
      size="lg"
    >
      <div class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Country</label>
            <div class="text-white">Vietnam</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">City</label>
            <div class="text-white">Ho Chi Minh City</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">District</label>
            <div class="text-white">District 1</div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-300 mb-1">Timezone</label>
            <div class="text-white">GMT+7</div>
          </div>
        </div>

        <div class="bg-gray-800 p-4 rounded-lg">
          <h4 class="font-medium text-white mb-2">Address</h4>
          <p class="text-gray-300">123 Demo Street, District 1, Ho Chi Minh City, Vietnam</p>
        </div>

        <div class="bg-gray-800 p-4 rounded-lg">
          <h4 class="font-medium text-white mb-2">Coordinates</h4>
          <p class="text-gray-300">Lat: 10.8231, Lng: 106.6297</p>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-location-detail')">
          Back to User Info
        </Button>
      </template>
    </Modal>

    <!-- Settings Modal (Standalone) -->
    <Modal
      id="modal-settings"
      title="Application Settings"
      :icon="Settings"
      icon-background="bg-gray-500/20"
      icon-color="text-gray-400"
      size="xl"
    >
      <div class="space-y-6">
        <div>
          <h3 class="text-lg font-semibold text-white mb-4">General Settings</h3>
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-sm font-medium text-gray-300">Dark Mode</label>
                <p class="text-xs text-gray-500">Toggle dark/light theme</p>
              </div>
              <input type="checkbox" checked class="rounded" />
            </div>
            <div class="flex items-center justify-between">
              <div>
                <label class="text-sm font-medium text-gray-300">Notifications</label>
                <p class="text-xs text-gray-500">Enable push notifications</p>
              </div>
              <input type="checkbox" checked class="rounded" />
            </div>
          </div>
        </div>

        <div class="border-t border-gray-700 pt-6">
          <h3 class="text-lg font-semibold text-white mb-4">Overlay Settings</h3>
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-sm font-medium text-gray-300">Close on Escape</label>
                <p class="text-xs text-gray-500">Close overlays with ESC key</p>
              </div>
              <input type="checkbox" checked class="rounded" />
            </div>
            <div class="flex items-center justify-between">
              <div>
                <label class="text-sm font-medium text-gray-300">Close on Outside Click</label>
                <p class="text-xs text-gray-500">Close when clicking outside overlay</p>
              </div>
              <input type="checkbox" checked class="rounded" />
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <Button variant="ghost" @click="closeOverlay('modal-settings')">
          Cancel
        </Button>
        <Button>
          <Save class="w-4 h-4 mr-2" />
          Save Settings
        </Button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  Users, UserPlus, Edit, Info, User, MapPin, Settings,
  Plus, Eye, Save
} from 'lucide-vue-next'
import { useOverlay } from '../composables/useOverlay'
import Drawer from './ui/Drawer.vue'
import Modal from './ui/Modal.vue'
import Button from './ui/Button.vue'

const {
  overlayStore,
  openOverlay,
  closeOverlay,
  closeAllOverlays,
  activeOverlay,
  hasActiveOverlay
} = useOverlay()

// Demo data
const demoUsers = ref([
  { id: 1, name: 'John Doe', email: 'john@example.com', role: 'admin' },
  { id: 2, name: 'Jane Smith', email: 'jane@example.com', role: 'user' },
  { id: 3, name: 'Bob Johnson', email: 'bob@example.com', role: 'moderator' }
])

const newUser = ref({
  name: '',
  email: '',
  role: 'user'
})

const editUser = ref({
  name: '',
  email: '',
  role: 'user'
})

const selectedUser = ref<any>(null)

// Watch for user selection in modals
watch(() => overlayStore.activeOverlayId, (newId) => {
  if (newId) {
    const overlay = overlayStore.getOverlayById(newId)
    if (overlay?.config.props?.user) {
      selectedUser.value = overlay.config.props.user
      editUser.value = { ...overlay.config.props.user }
    }
  }
})

const saveUser = () => {
  console.log('Saving user:', newUser.value)
  // Reset form
  newUser.value = { name: '', email: '', role: 'user' }
  closeOverlay('modal-add-user')
}

const updateUser = () => {
  console.log('Updating user:', editUser.value)
  closeOverlay('modal-edit-user')
}
</script>

<style scoped>
/* Additional styles if needed */
</style>
